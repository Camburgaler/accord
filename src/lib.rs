use eldenring::{
    cs::{CSChrPhysicsModule, CSTaskGroupIndex, CSTaskImp, WorldChrMan},
    fd4::FD4TaskData,
    position::HavokPosition,
    rotation::EulerAngles,
    util::system::wait_for_system_init,
};
use fromsoftware_shared::{F32Vector4, FromStatic, OwnedPtr, program::Program, task::*};
use mio::net::TcpStream;
use std::{
    net::SocketAddr,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use telemetry::{TelemetrySocket, TelemetryV1};

#[unsafe(no_mangle)]
/// # Safety
///
/// This is exposed this way such that windows LoadLibrary API can call it. Do not call this yourself.
pub unsafe extern "C" fn DllMain(_hmodule: usize, reason: u32) -> bool {
    // Check if the reason for the call is DLL_PROCESS_ATTACH.
    // This indicates that the DLL is being loaded into a process.
    if reason != 1 {
        return true;
    }

    // Kick off new thread.
    std::thread::spawn(|| {
        // Wait for game (current program we're injected into) to boot up.
        // This will block until the game initializes its systems (singletons, statics, etc).
        wait_for_system_init(&Program::current(), Duration::MAX)
            .expect("Could not await system init.");

        // Retrieve games task runner.
        let cs_task = unsafe { CSTaskImp::instance().unwrap() };

        let last_emit_ms = AtomicU64::new(0);
        let address: SocketAddr = "127.0.0.1:5555".parse().unwrap();
        let stream = TcpStream::connect(address).unwrap();

        // Create socket
        let socket = std::sync::Arc::new(Mutex::new(TelemetrySocket::new()));
        socket.lock().unwrap().set_stream(stream);
        let socket_bg = socket.clone();

        // Register a new task with the game to happen every frame during the gameloops
        // ChrIns_PostPhysics phase because all the physics calculations have ran at this
        // point.
        cs_task.run_recurring(
            // The registered task will be our closure.
            move |_: &FD4TaskData| {
                let now_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let prev = last_emit_ms.load(Ordering::Relaxed);

                if now_ms - prev >= 100 {
                    last_emit_ms.store(now_ms, Ordering::Relaxed);

                    // Grab the main player from WorldChrMan if it's available. Bail otherwise.
                    let Some(player) = unsafe { WorldChrMan::instance() }
                        .ok()
                        .and_then(|w| w.main_player.as_ref())
                    else {
                        return;
                    };

                    // Grab physics module from player.
                    let physics: &OwnedPtr<CSChrPhysicsModule> =
                        &player.chr_ins.module_container.physics;

                    // Send position and direction.
                    let pos: HavokPosition = physics.position;
                    let euler: EulerAngles = physics.orientation.to_euler_angles();
                    let chunk: F32Vector4 = player.chr_ins.chunk_position;

                    let delta_x: f32 = pos.0 - chunk.0;
                    let delta_y: f32 = pos.1 - chunk.1;
                    let delta_z: f32 = pos.2 - chunk.2;

                    let frame = TelemetryV1 {
                        timestamp_ms: now_ms,
                        pitch: euler.0,
                        yaw: euler.1,
                        roll: euler.2,
                        x: delta_x,
                        y: delta_y,
                        z: delta_z,
                    };

                    let bytes: &[u8] = unsafe {
                        std::slice::from_raw_parts(
                            &frame as *const TelemetryV1 as *const u8,
                            TelemetryV1::SIZE,
                        )
                    };

                    socket_bg.lock().unwrap().send(bytes);
                }
            },
            // Specify the task group in which physics calculations are already done.
            CSTaskGroupIndex::ChrIns_PostPhysics,
        );
    });

    // Signal that DllMain executed successfully
    true
}

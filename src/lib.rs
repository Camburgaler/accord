use eldenring::{
    cs::{
        CSChrPhysicsModule, CSTaskGroupIndex, CSTaskImp, ChrIns, FieldArea, WorldBlockInfo,
        WorldChrMan, WorldInfo, WorldInfoOwner, WorldRes,
    },
    fd4::FD4TaskData,
    util::system::wait_for_system_init,
};
use fromsoftware_shared::{FromStatic, OwnedPtr, program::Program, task::*};
use pelite::pe::Pe;
use std::{
    collections::VecDeque,
    io::Write,
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use telemetry::{FRAME_MAGIC, FrameHeader, FramePayload, RAW_TELEMETRY_ADDRESS, RawV1};

const RVA_GLOBAL_FIELD_AREA: u32 = 0x3d691d8;
const TICK_INTERVAL_MS: u64 = 100;

struct Outbox {
    queue: VecDeque<Vec<u8>>,
}

impl Outbox {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn push(&mut self, bytes: Vec<u8>) {
        self.queue.push_back(bytes);
    }

    fn pop(&mut self) -> Option<Vec<u8>> {
        self.queue.pop_front()
    }
}

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

    let outbox = Arc::new(Mutex::new(Outbox::new()));
    let outbox_bg = outbox.clone();

    std::thread::spawn(move || {
        let mut stream: Option<TcpStream> = None;

        loop {
            if stream.is_none() {
                stream = TcpStream::connect(RAW_TELEMETRY_ADDRESS).ok();
                if let Some(s) = &stream {
                    let _ = s.set_nodelay(true);
                }
            }

            if let Some(s) = stream.as_mut() {
                let maybe_buf = {
                    let mut ob = outbox_bg.lock().unwrap();
                    ob.pop()
                };

                if let Some(buf) = maybe_buf {
                    if let Err(e) = s.write_all(&buf) {
                        eprintln!("socket write failed: {e}");
                        stream = None; // force reconnect
                    }
                    // You can flush here if you want batching control:
                    // let _ = s.flush();
                } else {
                    std::thread::sleep(Duration::from_millis(1));
                }
            } else {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    });

    // Kick off new thread.
    std::thread::spawn(|| {
        let program: Program<'_> = Program::current();

        // Wait for game (current program we're injected into) to boot up.
        // This will block until the game initializes its systems (singletons, statics, etc).
        wait_for_system_init(&program, Duration::MAX).expect("Could not await system init.");

        // Retrieve games task runner.
        let cs_task: &mut CSTaskImp = unsafe { CSTaskImp::instance().unwrap() };

        let last_emit_ms: AtomicU64 = AtomicU64::new(0);
        // let socket: std::sync::Arc<Mutex<TelemetrySocket>> =
        //     std::sync::Arc::new(Mutex::new(TelemetrySocket::new()));
        // let socket_bg: std::sync::Arc<Mutex<TelemetrySocket>> = socket.clone();

        // std::thread::spawn(move || {
        //     loop {
        //         // Attempt connect WITHOUT holding the lock
        //         let new_stream: Option<TcpStream> = TcpStream::connect(RAW_TELEMETRY_ADDRESS).ok();

        //         if let Some(stream) = new_stream {
        //             let mut sock: std::sync::MutexGuard<'_, TelemetrySocket> =
        //                 match socket_bg.lock() {
        //                     Ok(guard) => guard,
        //                     Err(poisoned) => poisoned.into_inner(),
        //                 };
        //             if sock.stream.is_none() {
        //                 sock.set_stream(stream);
        //             }
        //         }

        //         std::thread::sleep(Duration::from_secs(5));
        //     }
        // });

        // Register a new task with the game to happen every frame during the gameloops
        // ChrIns_PostPhysics phase because all the physics calculations have ran at this
        // point.
        cs_task.run_recurring(
            // The registered task will be our closure.
            move |_: &FD4TaskData| {
                let now_ms: u64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let prev: u64 = last_emit_ms.load(Ordering::Relaxed);

                if now_ms - prev >= TICK_INTERVAL_MS {
                    last_emit_ms.store(now_ms, Ordering::Relaxed);

                    // Grab the main player from WorldChrMan if it's available. Bail otherwise.
                    let Some(player) = unsafe { WorldChrMan::instance() }
                        .ok()
                        .and_then(|w: &mut WorldChrMan| w.main_player.as_ref())
                    else {
                        return;
                    };

                    // Grab physics module from player.
                    let chr_ins: &ChrIns = &player.chr_ins;
                    let physics: &OwnedPtr<CSChrPhysicsModule> = &chr_ins.module_container.physics;

                    // Grab world block info.
                    let world_block_info: &WorldBlockInfo;
                    if let Some(field_area) = unsafe {
                        (*(program.rva_to_va(RVA_GLOBAL_FIELD_AREA).unwrap()
                            as *const *const FieldArea))
                            .as_ref()
                    } {
                        let world_info_owner: &WorldInfoOwner = &field_area.world_info_owner;
                        let world_res: &WorldRes = &world_info_owner.world_res;
                        let world_info: &WorldInfo = &world_res.world_info;
                        world_block_info =
                            match world_info.world_block_info_by_map(&chr_ins.block_id) {
                                Some(b) => b,
                                None => return,
                            }
                    } else {
                        return;
                    }

                    let frame_header: FrameHeader = FrameHeader {
                        magic: FRAME_MAGIC,
                        payload_size: RawV1::WIRE_SIZE,
                        frame_type: RawV1::FRAME_TYPE,
                        version: RawV1::VERSION,
                        timestamp_ms: now_ms,
                    };

                    // Send position and direction.
                    let frame: RawV1 = RawV1 {
                        yaw: physics.orientation.to_euler_angles().1,
                        position: physics.position,
                        initial_position: chr_ins.initial_position,
                        chunk_position: chr_ins.chunk_position,
                        block_center: world_block_info.physics_center,
                        block_id: chr_ins.block_id,
                    };

                    let bytes: Vec<u8> = frame.to_bytes();

                    let mut packet: Vec<u8> = Vec::new();
                    // frame_header.write_to(&mut packet);
                    packet.extend_from_slice(&frame_header.to_bytes());
                    packet.extend_from_slice(&bytes);

                    if let Ok(mut ob) = outbox.lock() {
                        ob.push(packet);
                    }
                }
            },
            // Specify the task group in which physics calculations are already done.
            CSTaskGroupIndex::ChrIns_PostPhysics,
        );
    });

    // Signal that DllMain executed successfully
    true
}

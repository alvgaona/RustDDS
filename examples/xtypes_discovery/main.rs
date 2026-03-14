use std::time::Duration;

use futures::StreamExt;
use rustdds::{DomainParticipantBuilder, DomainParticipantStatusEvent, StatusEvented};
use smol::Timer;

fn main() {
    env_logger::init();

    let domain_id: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    println!("XTypes Discovery Example - Domain {domain_id}");
    println!("Listening for endpoints with TypeInformation...");
    println!("Run a ROS 2 publisher in another terminal, e.g.:");
    println!("  ros2 topic pub /hello std_msgs/msg/String '{{data: hello}}'");
    println!();

    let domain_participant = DomainParticipantBuilder::new(domain_id)
        .build()
        .unwrap_or_else(|e| panic!("DomainParticipant construction failed: {e:?}"));

    smol::block_on(async {
        let dp_status_listener = domain_participant.status_listener();
        let mut dp_status_stream = dp_status_listener.as_async_status_stream();
        let timeout = futures::FutureExt::fuse(Timer::after(Duration::from_secs(30)));
        futures::pin_mut!(timeout);

        loop {
            futures::select! {
                _ = timeout => {
                    println!("Timeout after 30s. Exiting.");
                    break;
                }
                event = dp_status_stream.select_next_some() => {
                    match event {
                        DomainParticipantStatusEvent::WriterDetected { writer } => {
                            println!("--- Writer Detected ---");
                            println!("  GUID:  {:?}", writer.guid);
                            println!("  Topic: {}", writer.topic_name);
                            println!("  Type:  {}", writer.type_name);
                            match &writer.type_information {
                                Some(ti) => {
                                    println!("  TypeInformation: YES");
                                    let minimal_id = &ti.minimal.typeid_with_size.type_id;
                                    let complete_id = &ti.complete.typeid_with_size.type_id;
                                    println!("    Minimal TypeId:  {minimal_id:?}");
                                    println!("    Complete TypeId: {complete_id:?}");
                                    println!(
                                        "    Minimal TypeObject size:  {}",
                                        ti.minimal.typeid_with_size.typeobject_serialized_size
                                    );
                                    println!(
                                        "    Complete TypeObject size: {}",
                                        ti.complete.typeid_with_size.typeobject_serialized_size
                                    );
                                    let dep_count = ti.complete.dependent_typeids.len();
                                    println!("    Complete dependencies: {dep_count}");
                                    for (i, dep) in ti.complete.dependent_typeids.iter().enumerate() {
                                        println!("      [{i}] {:?} (size: {})", dep.type_id, dep.typeobject_serialized_size);
                                    }
                                }
                                None => {
                                    println!("  TypeInformation: NONE");
                                }
                            }
                            println!();
                        }
                        DomainParticipantStatusEvent::ReaderDetected { reader } => {
                            println!("--- Reader Detected ---");
                            println!("  GUID:  {:?}", reader.guid);
                            println!("  Topic: {}", reader.topic_name);
                            println!("  Type:  {}", reader.type_name);
                            match &reader.type_information {
                                Some(ti) => {
                                    println!("  TypeInformation: YES");
                                    let complete_id = &ti.complete.typeid_with_size.type_id;
                                    println!("    Complete TypeId: {complete_id:?}");
                                }
                                None => {
                                    println!("  TypeInformation: NONE");
                                }
                            }
                            println!();
                        }
                        other => {
                            println!("Event: {other:?}");
                        }
                    }
                }
            }
        }
    });
}

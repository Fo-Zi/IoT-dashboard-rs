use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::error::Error;
use std::time::Duration;
use tokio::{sync::mpsc, task};

#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    
    let ui = AppWindow::new()?;
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });
    ui.run()?;

    // Initialize MQTT client
    let mut mqttoptions = MqttOptions::new("test-1", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe("test/topic", QoS::AtMostOnce)
        .await
        .unwrap();

    // Create a channel to transfer messages between threads
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn a task to handle message receiving
    task::spawn(async move {
        while let Ok(event) = eventloop.poll().await {
            if let Event::Incoming(Packet::Publish(publish)) = event {
                let message = String::from_utf8_lossy(&publish.payload).to_string();
                if (tx.send((publish.topic.clone(), message)).await).is_err() {
                    println!("Receiver dropped. Exiting message handler.");
                    break;
                }
            }
        }
    });

    // Main thread processing received messages
    while let Some(msg) = rx.recv().await {
        println!("Received message - Topic:{} - Message: {}", msg.0, msg.1);
    }

    Ok(())

}

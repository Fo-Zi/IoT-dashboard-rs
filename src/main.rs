use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::error::Error;
use std::time::Duration;
use tokio::{sync::mpsc, task};

#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the UI
    let ui = AppWindow::new()?;

    // Initialize MQTT client
    let mut mqttoptions = MqttOptions::new("test-1", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe("iot_test", QoS::AtMostOnce)
        .await
        .unwrap();

    // Create a channel to transfer messages between threads
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn a task to handle MQTT message receiving
    task::spawn(async move {
        while let Ok(event) = eventloop.poll().await {
            if let Event::Incoming(Packet::Publish(publish)) = event {
                let message = String::from_utf8_lossy(&publish.payload).to_string();
                if tx.send((publish.topic.clone(), message)).await.is_err() {
                    println!("Receiver dropped. Exiting message handler.");
                    break;
                }
            }
        }
    });
     // now forward the data to the main thread using invoke_from_event_loop
    
    // Main thread processing received messages and updating the UI
    let ui_handle = ui.as_weak();
    task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("Received message - Topic:{} - Message: {}", msg.0, msg.1);
            let message = msg.1.clone();
            let handle_copy = ui_handle.clone();
            slint::invoke_from_event_loop(move || handle_copy.unwrap().set_temperature_display(message.into()));
        }
    });

    // Run the UI in the main thread
    ui.run()?;
    Ok(())

}

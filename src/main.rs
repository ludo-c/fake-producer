use event_listener::Event;
use futures::channel::mpsc;
use futures::SinkExt;
use futures::StreamExt;
use std::error::Error;
use timer::Timer;
use zbus::dbus_interface;
use zbus::ConnectionBuilder;
use zbus::SignalContext;

struct Greeter {
    done: Event,
}

const BUS_NAME: &str = "ludo_ic.daemon.producer";
const INTERFACE_NAME: &str = "/ludo_ic/daemon/producer";
const INTERNAL_TIMER: i64 = 1;

#[dbus_interface(name = "ludo_ic.daemon.producer")]
impl Greeter {
    async fn say_hello(&self, name: &str) -> String {
        format!("Hello {}!", name)
    }

    #[dbus_interface(signal)]
    async fn MySignalEvent(
        ctxt: &SignalContext<'_>,
        val1: i32,
        val2: i32,
    ) -> Result<(), zbus::Error>;
}

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let greeter = Greeter {
        done: event_listener::Event::new(),
    };
    let done_listener = greeter.done.listen();
    let conn = ConnectionBuilder::session()?
        .name(BUS_NAME)?
        .serve_at(INTERFACE_NAME, greeter)?
        .build()
        .await?;

    let t = Timer::new();

    let (mut tx, mut rx) = mpsc::unbounded();
    let _timer_guard =
        t.schedule_repeating(chrono::Duration::milliseconds(INTERNAL_TIMER), move || {
            async_std::task::block_on(async {
                tx.send(1).await.map_err(|e| println!("error: {}", e)).ok();
            })
        });

    async_std::task::spawn(async move {
        let iface = conn
            .object_server()
            .interface::<_, Greeter>(INTERFACE_NAME)
            .await
            .unwrap();
        let sc = iface.signal_context();
        while let Some(_) = rx.next().await {
            //println!("unblocked !");
            Greeter::MySignalEvent(sc, 1, 43).await.unwrap();
        }
    });

    done_listener.wait();

    Ok(())
}

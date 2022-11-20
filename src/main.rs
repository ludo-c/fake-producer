use std::error::Error;
use zbus::dbus_interface;
use zbus::SignalContext;
use zbus::{ConnectionBuilder, InterfaceRef};

struct Greeter {}

const BUS_NAME: &str = "ludo_ic.daemon.producer";
const INTERFACE_NAME: &str = "/ludo_ic/daemon/producer";
const INTERNAL_TIMER: u64 = 1;

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
    let greeter = Greeter {};
    let conn = ConnectionBuilder::session()?
        .name(BUS_NAME)?
        .serve_at(INTERFACE_NAME, greeter)?
        .build()
        .await?;

    let iface: InterfaceRef<Greeter> = conn
        .object_server()
        .interface(INTERFACE_NAME)
        .await
        .unwrap();
    let sc = iface.signal_context();

    loop {
        async_std::task::sleep(std::time::Duration::from_millis(INTERNAL_TIMER)).await;
        //println!("unblocked !");
        Greeter::MySignalEvent(sc, 1, 43).await.unwrap();
    }
}

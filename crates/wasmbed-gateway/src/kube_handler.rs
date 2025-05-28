// use kube::{Api, Client};
// use kube::api::ListParams;
// use tokio::sync::{Channel, mpsc};
//
// enum WasmbedEvents {
//     CreateApplication(Application),
//     DeleteApplication(Application),
//     PairingStatus(bool),
//     RegisterDevice(Device),
// }
//
// struct WasmbedClient {
//     client: Client,
//     sender: Sender<Vec<u8>>,
// }
//
// impl WasmbedClient {
//     pub fn application_deployment_handler(&self) {
//         todo!()
//     }
//
//     pub fn find_device(&self) -> DeviceId {
//         todo!()
//     }
//
//     pub fn register_device(&self) {
//         todo!()
//     }
//
//     pub fn get_gateway_pairing(&self) -> bool {
//         todo!()
//     }
// }

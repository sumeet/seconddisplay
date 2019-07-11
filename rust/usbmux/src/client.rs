use std::time::Duration;
use plist::Plist;

use Stream;
use Result;
use Error;
use message_type;

/// A Client for usbmuxd.
pub struct Client {
    stream: Stream,
}

impl Client {
    /// Tries to create a new instance of the `Client`.
    pub fn new() -> Result<Self> {
        let mut stream = try!(Stream::connect());
        try!(stream.set_send_tymeout(Some(Duration::new(1, 0))));
        try!(stream.set_receive_timeout(Some(Duration::new(1, 0))));
        Ok(Client {
            stream: stream,
        })
    }

    /// Returns a list of connected devices.
    pub fn devices(&mut self) -> Result<Vec<Device>> {
        let mut plist = try!(self.stream.request(Plist::Dictionary(message_type("ListDevices"))));
        let dict = try!(plist.as_dictionary_mut().ok_or(Error::UnexpectedFormat));
        match dict.remove("DeviceList") {
            Some(Plist::Array(array)) => {
                let results = array.into_iter().filter_map(|mut item| {
                    item.as_dictionary_mut().and_then(|dict| {
                        dict.remove("Properties").and_then(|plist| {
                            Device::from_plist(plist)
                        })
                    })
                }).collect();
                Ok(results)
            },
            _ => Err(Error::UnexpectedFormat),
        }
    }

    /// Returns `Stream` connected to a `port` of the device.
    pub fn connection(mut self, device_id: u32, port: u16) -> Result<Stream> {
        fn byte_swap(v: u16) -> u16 {
            ((v & 0xFF) << 8) | ((v >> 8) & 0xFF)
        }

        let mut message = message_type("Connect");
        message.insert("DeviceID".to_owned(), Plist::Integer(device_id as i64));
        message.insert("PortNumber".to_owned(), Plist::Integer(byte_swap(port) as i64));
        let plist = try!(self.stream.request(Plist::Dictionary(message)));
        match plist.as_dictionary() {
            Some(dict) => {
                match dict.get("Number") {
                    Some(&Plist::Integer(i)) => {
                        match i {
                            0 => Ok(self.stream),
                            2 => Err(Error::DeviceIsNotConnected),
                            3 => Err(Error::PortIsNotAvailable),
                            _ => Err(Error::UnexpectedFormat),
                        }
                    },
                    _ => Err(Error::UnexpectedFormat),
                }
            },
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

/// Represents a device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub device_id: u32,
    pub product_id: u32,
    pub location_id: u32,
    pub serial_number: String,
}

impl Device {
    /// Creates an instance of `Device` from plist.
    pub fn from_plist(mut plist: Plist) -> Option<Device> {
        let dict = try_opt!(plist.as_dictionary_mut());
        Some(Device {
            device_id: try_opt!(dict.get("DeviceID").and_then(Plist::as_integer).map(|x| x as u32)),
            product_id: try_opt!(dict.get("ProductID").and_then(Plist::as_integer).map(|x| x as u32)),
            location_id: try_opt!(dict.get("LocationID").and_then(Plist::as_integer).map(|x| x as u32)),
            serial_number: try_opt!(dict.remove("SerialNumber").and_then(Plist::into_string)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;
    use plist::Plist;
    use std::collections::BTreeMap;

    #[test]
    fn test_device_from_plist() {
        let mut map = BTreeMap::new();
        map.insert("ConnectionSpeed".to_owned(), Plist::Integer(480000000));
        map.insert("ConnectionType".to_owned(), Plist::String("USB".to_owned()));
        map.insert("DeviceID".to_owned(), Plist::Integer(3));
        map.insert("LocationID".to_owned(), Plist::Integer(336592896));
        map.insert("ProductID".to_owned(), Plist::Integer(4778));
        map.insert("SerialNumber".to_owned(),Plist::String("fffffffff".to_owned()));

        let device = Device {
            device_id: 3,
            product_id: 4778,
            location_id: 336592896,
            serial_number: "fffffffff".to_owned(),
        };

        expect!(Device::from_plist(Plist::Dictionary(map))).to(be_some().value(device));
    }
}
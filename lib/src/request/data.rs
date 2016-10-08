use std::io::Read;

pub struct Data {
    stream: Box<Read>
}

pub trait FromData {
    fn from_data(data: Data) -> Outcome {  }
}

// TODO: Allow streamed responses.
// const CHUNK_SIZE: u32 = 4096;
// pub struct Stream<T: Read>(T);
// impl<T> Responder for Stream<T> {
//     fn respond<'a>(&self, mut r: HypResponse<'a, HypFresh>) {
//         r.headers_mut().set(header::TransferEncoding(vec![Encoding::Chunked]));
//         *(r.status_mut()) = StatusCode::Ok;
//         let mut stream = r.start();

//         r.write()
//         Response {
//             status: StatusCode::Ok,
//             headers: headers,
//             body: Body::Stream(r)
//         }
//     }
// }

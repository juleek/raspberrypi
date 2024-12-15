#[derive(Debug)]
struct RouteGuideService;

type Stream = dyn futures::Stream<Item = Result<proto::CounterResp, tonic::Status>> + Send;
type PBStream = std::pin::Pin<Box<Stream>>;

#[tonic::async_trait]
impl proto::route_guide_server::RouteGuide for RouteGuideService {
   type SendMessageStream = PBStream;

   async fn get_feature(&self,
                        _request: tonic::Request<proto::CounterReq>)
                        -> Result<tonic::Response<proto::CounterResp>, tonic::Status> {
      println!("Request from client = {:?}", _request);
      let resp = proto::CounterResp { counter: 2 };
      return Ok(tonic::Response::new(resp));
   }


   async fn send_message(&self,
                         request: tonic::Request<tonic::Streaming<proto::CounterReq>>)
                         -> Result<tonic::Response<Self::SendMessageStream>, tonic::Status> {
      // converting request in stream
      let mut stream = request.into_inner();

      use futures::StreamExt;
      let output = async_stream::try_stream! {
                      while let Some(req) = stream.message().await.unwrap_or(None) {
                         println!("Server received: {:?}", req);
                         let response = proto::CounterResp { counter: req.counter + 1, };
                         println!("Server sending: {:?}", response);
                         yield response;
                         }
                   }.boxed();


      Ok(tonic::Response::new(output as PBStream))
   }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   let addr = "[::1]:10000".parse().unwrap();
   let route_guide = RouteGuideService {};
   let svc = proto::route_guide_server::RouteGuideServer::new(route_guide);
   tonic::transport::Server::builder().add_service(svc)
                                      .serve(addr)
                                      .await?;
   Ok(())
}

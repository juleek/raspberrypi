
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = proto::route_guide_client::RouteGuideClient::connect("http://[::1]:10000").await?;
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let outbound_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let mut inbound_stream = client.send_message(tonic::Request::new(outbound_stream)).await?.into_inner();
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let mut i = 0;
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let request = proto::CounterReq { counter: i };
                i += 1;
                println!("Client sending: {:?}", request);

                if tx.send(request).await.is_err() {
                    println!("Error");
                    break;
                }
            },
            response = inbound_stream.message() => {
               println!("Client received from server: {:?}", response);
            },
        }
    }
    Ok(())
}

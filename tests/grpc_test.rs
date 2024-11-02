use rust_http_template::grpc::hello_world::helloworld::{
    greeter_client::GreeterClient, HelloRequest,
};

#[tokio::test]
async fn test_grpc_hello() -> Result<(), Box<dyn std::error::Error>> {
    // Create a gRPC client
    let mut client = GreeterClient::connect("http://0.0.0.0:50051").await?;

    // Create a new request
    let request = tonic::Request::new(HelloRequest {
        name: "World".to_string(),
    });

    // Send the request and await the response
    let response = client.say_hello(request).await?;
    assert_eq!(response.get_ref().message, "Hello World!");
    println!("RESPONSE={:?}", response);

    Ok(())
}

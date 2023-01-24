use crate::sink;
use crate::jwt_updater;
use crossbeam_channel as channel;

struct HttpSink {
    jwt_channel: channel::Receiver<jwt_updater::ReqResp::JWT>,
}

impl sink::Sink for HttpSink {
   fn publish(&mut self, item: sink::Item) {}
}

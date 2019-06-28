use std::borrow::Borrow;
use std::error::Error;

pub trait Action {
    fn run(&mut self, client: &slack::RtmClient, event: &slack::Event);
}

pub struct Response {
    re: regex::Regex,
    template: mustache::Template,
}

impl Response {
    pub fn new(re: &str, template: &str) -> Result<Self, String> {
        Ok(Self {
            re: regex::Regex::new(re).map_err(|error| {
                "Could not compile trigger regex!".to_string() + error.description()
            })?,
            template: mustache::compile_str(template).map_err(|error| {
                "Could not compile response template! ".to_string() + error.description()
            })?,
        })
    }
}

impl Action for Response {
    fn run(&mut self, client: &slack::RtmClient, event: &slack::Event) {
        if let slack::Event::Message(message) = event {
            if let slack::Message::Standard(message_standard) = message.borrow() {
                let channel = message_standard.channel.as_ref().unwrap();
                let text = message_standard.text.as_ref().unwrap();
                if let Some(captures) = self.re.captures(&text) {
                    let mut builder = mustache::MapBuilder::new();
                    for name in self.re.capture_names() {
                        if let Some(name) = name {
                            if let Some(value) = captures.name(name) {
                                builder = builder.insert_str(name, value.as_str());
                            }
                        }
                    }
                    let data = builder.build();
                    if let Ok(response) = self.template.render_data_to_string(&data) {
                        let _ = client.sender().send_message(&channel, &response);
                    }
                }
            }
        }
    }
}

mod actions;

use std::borrow::Borrow;

pub struct Handler {
    bot_id: String,
    commands_re: regex::Regex,
    actions: std::collections::HashMap<String, Box<actions::Action>>,
}

impl slack::EventHandler for Handler {
    fn on_event(&mut self, client: &slack::RtmClient, event: slack::Event) {
        println!("on_event(event: {:?})", event);

        if !self.parse_command(client, &event) {
            for action in self.actions.values_mut() {
                action.run(client, &event);
            }
        }
    }

    fn on_close(&mut self, client: &slack::RtmClient) {}

    fn on_connect(&mut self, client: &slack::RtmClient) {}
}

impl Handler {
    pub fn new(cli: &slack::RtmClient) -> Self {
        let bot_id = cli
            .start_response()
            .slf
            .as_ref()
            .unwrap()
            .id
            .as_ref()
            .unwrap();
        Self {
            bot_id: bot_id.to_string(),
            commands_re: regex::Regex::new(
                        (format!(r"{}\W+(?:", regex::escape(bot_id)) + 
                        concat!(
                        r"(?P<addresponse>add-response)\W+(?P<addresponsename>\w+)\W+<<(?P<trigger>.*)>>\W+<<(?P<response>.*)>>|",
                        r"(?P<removeresponse>remove-response)\W+(?P<removeresponsename>\w+)|",
                        r"(?P<listresponses>list-responses)",
                        r")"
                        )).as_str()
                )
            .unwrap(),
            actions: std::collections::HashMap::new(),
        }
    }

    fn parse_command(&mut self, client: &slack::RtmClient, event: &slack::Event) -> bool {
        if let slack::Event::Message(message) = event {
            if let slack::Message::Standard(message_standard) = message.borrow() {
                let channel = message_standard.channel.as_ref().unwrap();
                let text =
                    htmlescape::decode_html(message_standard.text.as_ref().unwrap()).unwrap();

                // Ignore messages sent by Reggie
                if message_standard.user.as_ref().unwrap() == &self.bot_id {
                    return false;
                }

                // Check for a command
                if let Some(capture) = self.commands_re.captures(text.as_str()) {
                    if capture.name("addresponse").is_some() {
                        let name = capture.name("addresponsename").unwrap().as_str();
                        let trigger = capture.name("trigger").unwrap().as_str();
                        let response = capture.name("response").unwrap().as_str();
                        if self.actions.contains_key(name) {
                            let _ = client.sender().send_message(
                                &channel,
                                format!("An action '{}' already exists!", name).as_str(),
                            );
                        } else {
                            match actions::Response::new(trigger, response) {
                                Ok(action) => {
                                    println!("Adding response: {}", name);
                                    self.actions.insert(name.to_string(), Box::new(action));
                                }
                                Err(error) => {
                                    client.sender().send_message(&channel, error.as_str());
                                }
                            }
                        }
                    } else if capture.name("removeresponse").is_some() {
                        let name = capture.name("removeresponsename").unwrap().as_str();
                        if self.actions.contains_key(name) {
                            self.actions.remove(name);
                        } else {
                            client.sender().send_message(
                                &channel,
                                format!("No action '{}' to remove!", name).as_str(),
                            );
                        }
                    } else if capture.name("listresponses").is_some() {
                        client.sender().send_message(
                            &channel,
                            format!(
                                "Actions: {:?}",
                                self.actions.keys().collect::<Vec<&String>>()
                            )
                            .as_str(),
                        );
                    }
                    return true;
                }
            }
        }
        false
    }
}

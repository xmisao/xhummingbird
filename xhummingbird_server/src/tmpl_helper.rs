pub fn filter_link(title: Option<String>, service: Option<String>) -> String {
    let mut encoded = form_urlencoded::Serializer::new(String::new());

    if title.is_some() {
        encoded.append_pair("title", &title.unwrap());
    }
    if service.is_some() {
        encoded.append_pair("service", &service.unwrap());
    }

    format!("/events?{}", encoded.finish())
}

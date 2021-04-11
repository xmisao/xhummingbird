pub fn title_to_title_link(title: &String) -> String {
    let mut encoded = form_urlencoded::Serializer::new(String::new());
    encoded.append_pair("title", title);

    format!("/events?{}", encoded.finish())
}

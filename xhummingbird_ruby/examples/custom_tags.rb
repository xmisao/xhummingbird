require 'xhummingbird'

raise "Set #{Xhummingbird::Client::XH_SERVER} environment variable" unless Xhummingbird.enabled?

Xhummingbird.start

Xhummingbird.send_trace(
  title: "CustomTagTrace",
  message: "CustomTags",
  tags: {
    tag_a: 42,
    tag_b: "foo",
    tag_c: :bar
  }
)

begin
  raise 'Something wrong'
rescue => e
  Xhummingbird.send_exception(e, tags: {path: __FILE__})
end

sleep 1 # Await sending

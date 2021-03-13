require 'xhummingbird'

raise "Set #{Xhummingbird::Client::XH_SERVER} environment variable" unless Xhummingbird.enabled?

Xhummingbird.send_trace(title: "SampleTrace", message: "Something happend")

sleep 1 # Await sending

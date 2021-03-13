require 'xhummingbird'

raise "Set #{Xhummingbird::Client::XH_SERVER} environment variable" unless Xhummingbird.enabled?

threads = 10.times.map do |i|
  Thread.start do
    Xhummingbird.send_trace(title: "Thread", message: "Send trace from thread #{i}")
  end
end

threads.map(&:join)

sleep 1 # Await sending

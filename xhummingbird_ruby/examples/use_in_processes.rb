require 'xhummingbird'

raise "Set #{Xhummingbird::Client::XH_SERVER} environment variable" unless Xhummingbird.enabled?

Xhummingbird.send_trace(title: "ParentProcess", message: "Send trace from parent process")

process_ids = 10.times.map do |i|
  fork do
    Xhummingbird.send_trace(title: "ChildProcess", message: "Send trace from child process #{i}")

    sleep 1 # Await sending
  end
end

process_ids.each do |pid|
  Process.waitpid(pid)
end

require 'xhummingbird'
require 'socket'

raise "Set #{Xhummingbird::Client::XH_SERVER} environment variable" unless Xhummingbird.enabled?

Xhummingbird.send(
  level: 1,
  title: "Connect before fork",
  message: "Sent event from main process",
  trace: [],
  tags: {hostname: Socket.gethostname},
  timestamp: Time.now
)

process_ids = 10.times.map do |i|
  fork do
    begin
      raise 'Something wrong'
    rescue => e
      puts "Sending... #{i}"

      Xhummingbird.send(
        level: 5,
        title: e.class.name,
        message: e.message,
        trace: e.backtrace,
        tags: {hostname: Socket.gethostname},
        timestamp: Time.now
      )

      sleep 1 # Await sending
    end
  end
end

process_ids.each do |pid|
  Process.waitpid(pid)
end

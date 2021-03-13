require 'xhummingbird'
require 'socket'

raise "Set #{Xhummingbird::Client::XH_SERVER} environment variable" unless Xhummingbird.enabled?

threads = 10.times.map do |i|
  Thread.start do
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
    end
  end
end

threads.map(&:join)

sleep 1 # Await sending

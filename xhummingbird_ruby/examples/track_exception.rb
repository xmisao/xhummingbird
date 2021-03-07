require 'xhummingbird'
require 'socket'

raise "Set #{Xhummingbird::Client::XH_SERVER} environment variable" unless Xhummingbird.enabled?

10.times do |i|
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

  sleep 1
end

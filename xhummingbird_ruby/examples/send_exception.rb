require 'xhummingbird'

raise "Set #{Xhummingbird::Client::XH_SERVER} environment variable" unless Xhummingbird.enabled?

begin
  raise 'Something wrong'
rescue => e
  Xhummingbird.send_exception(e)
end

sleep 1 # Await sending

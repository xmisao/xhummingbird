require 'xhummingbird'
require 'xhummingbird/short_name'

raise "Set #{XH::Client::XH_SERVER} environment variable" unless XH.enabled?

XH.send_trace(title: "ShortNameTrace", message: "Send trace from short name reference")

begin
  raise 'Something wrong'
rescue => e
  XH.send_exception(e)
end

sleep 1 # Await sending

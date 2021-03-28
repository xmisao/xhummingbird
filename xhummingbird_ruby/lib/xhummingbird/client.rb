module Xhummingbird
  class Client
    include Singleton

    XH_SERVER = 'XH_SERVER'

    def initialize
      @pid = nil
      @socket = nil
      @active = false
    end

    def send(message)
      if active?
        @socket.send_string(message)
      else
        Xhummingbird.debug("Xhummingbird not started.")
      end
    end

    def enabled?
      @enabled ||= !!address
    end

    def start
      ctx = ZMQ::Context.new
      socket = ctx.socket(ZMQ::PUSH)
      socket.connect(address)
      @socket = socket
      Xhummingbird.debug("Socket created (pid: #{$$})")

      at_exit do
        Xhummingbird.debug("at_exit started.")
        @socket.close
        Xhummingbird.debug("at_exit stopped.")
      end

      @pid = $$
      @active = true
    end

    private

    def address
      ENV[XH_SERVER]
    end

    def active?
      @pid == $$ && @socket && @active
    end
  end
end

module Xhummingbird
  class Client
    include Singleton

    XH_SERVER = 'XH_SERVER'

    def send(message)
      socket.send_string(message)
    end

    def enabled?
      @setup ||= !!address
    end

    private

    def socket
      return @socket if defined? @socket

      ctx = ZMQ::Context.new
      @socket = ctx.socket(ZMQ::PUB)
      @socket.connect(address)

      @socket
    end

    def address
      ENV[XH_SERVER]
    end
  end
end

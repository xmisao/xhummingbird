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
      return @socket if defined?(@socket) && @pid == Process.pid

      @socket = init_socket
    end

    def init_socket
      mutex.synchronize do
        return @socket if defined?(@socket) && @pid == Process.pid

        @pid = Process.pid

        ctx = ZMQ::Context.new
        socket = ctx.socket(ZMQ::PUSH)
        socket.connect(address)

        @socket = socket
      end
    end

    def address
      ENV[XH_SERVER]
    end

    def mutex
      @mutex ||= Mutex.new
    end
  end
end

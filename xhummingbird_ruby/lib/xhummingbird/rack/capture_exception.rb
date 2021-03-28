module Xhummingbird
  module Rack
    class CaptureException
      CONVERT_KEYS = Set.new(%w(
        REQUEST_METHOD
        SCRIPT_NAME
        PATH_INFO
        QUERY_STRING
        SERVER_NAME
        SERVER_PORT
        rack.version
        rack.url_scheme
        rack.multithread
        rack.multiprocess
        rack.run_once
        rack.hijack?
      ))

      HTTP_HEADER_PREFIX = 'HTTP_'

      def initialize(app)
        @app = app
      end

      def call(env)
        begin
          response = @app.call(env)
        rescue => e
          Xhummingbird.send_exception(e, tags: convert_to_rack_tags(env))
        end

        error = env['rack.exception'] || env['sinatra.error']

        Xhummingbird.send_exception(error, tags: convert_to_rack_tags(env)) if error

        response
      end

      private

      def convert_to_rack_tags(env)
        tags = {}

        env.each do |k, v|
          if CONVERT_KEYS.include?(k) || k.start_with?(HTTP_HEADER_PREFIX)
            tags["rack_env/" +  k.to_s] = v.to_s rescue nil
          end
        end

        tags
      end
    end
  end
end

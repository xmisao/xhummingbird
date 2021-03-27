module Xhummingbird
  module Rack
    class CaptureException
      def initialize(app)
        @app = app
      end

      def call(env)
        p env

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
          tags["rack_env/" +  k.to_s] = v.to_s rescue nil
        end

        tags
      end
    end
  end
end

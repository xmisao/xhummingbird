# frozen_string_literal: true

require 'singleton'
require 'ffi-rzmq'
require 'socket'
require 'logger'
require 'set'

require_relative "xhummingbird/version"
require_relative "xhummingbird/client"
require_relative "xhummingbird/protos/event_pb"

module Xhummingbird
  class Error < StandardError; end

  XH_SERVICE = 'XH_SERVICE'

  LOGGER = Logger.new(STDERR)

  def self.debug(*args)
    LOGGER.debug(*args) if ENV['XH_DEBUG']
  end

  def self.start
    debug(__method__)

    Client.instance.start

    send_trace(title: "Started", message: "Xhummingbird Ruby SDK started.", level: 0)
  end

  def self.send_trace(title:, message: "", level: 1, tags: {}, service: nil)
    debug(__method__)

    return unless enabled?

    send(
      level: level,
      title: title.to_s,
      message: message.to_s,
      trace: caller,
      tags: default_tags.merge(format_hash(tags)),
      timestamp: Time.now,
      service: service || default_service
    )
  rescue => e
    debug(e)
  end

  def self.send_exception(exception, level: 2, tags: {}, service: nil)
    debug(__method__)

    return unless enabled?

    send(
      level: level,
      title: exception.class.name,
      message: exception.message,
      trace: exception.backtrace,
      tags: default_tags.merge(format_hash(tags)),
      timestamp: Time.now,
      service: service || default_service
    )
  rescue => e
    debug(e)
  end

  def self.send_event(**args)
    debug(__method__)

    return unless enabled?

    send(**args)
  rescue => e
    debug(e)
  end

  def self.enabled?
    debug(__method__)

    Client.instance.enabled?
  end

  def self.default_tags
    debug(__method__)

    {
      "default/sdk" => "Ruby #{Xhummingbird::VERSION}",
      "default/hostname" =>  Socket.gethostname,
      "default/ruby_version" =>  RUBY_VERSION,
      "default/pid" =>  Process.pid.to_s,
      "default/thread_object_id" => Thread.current.object_id.to_s
    }
  end

  def self.send(**args)
    debug(__method__)

    event = Event.new(**args)
    message = Event.encode(event)
    Client.instance.send(message)
  end

  def self.format_hash(hash)
    debug(__method__)

    formatted = {}

    hash.each do |k, v|
      formatted[k.to_s] = v.to_s
    end

    formatted
  end

  def self.default_service
    debug(__method__)

    ENV.fetch(XH_SERVICE, "")
  end
end

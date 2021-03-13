# frozen_string_literal: true

require 'singleton'
require 'ffi-rzmq'
require 'socket'

require_relative "xhummingbird/version"
require_relative "xhummingbird/client"
require_relative "xhummingbird/protos/event_pb"

module Xhummingbird
  class Error < StandardError; end

  def self.send_trace(title:, message: "", level: 1)
    return unless enabled?

    send(
      level: level,
      title: title.to_s,
      message: message.to_s,
      trace: caller,
      tags: default_tags,
      timestamp: Time.now
    )
  rescue
    raise Error
  end

  def self.send_exception(exception, level: 2)
    return unless enabled?

    send(
      level: level,
      title: exception.class.name,
      message: exception.message,
      trace: exception.backtrace,
      tags: default_tags,
      timestamp: Time.now
    )
  rescue
    raise Error
  end

  def self.send_event(**args)
    return unless enabled?

    send(**args)
  rescue
    raise Error
  end

  def self.enabled?
    Client.instance.enabled?
  end

  def self.default_tags
    {
       hostname: Socket.gethostname,
       ruby_version: RUBY_VERSION,
       pid: Process.pid.to_s,
       thread_object_id: Thread.current.object_id.to_s
    }
  end

  def self.send(**args)
    event = Event.new(**args)
    message = Event.encode(event)
    Client.instance.send(message)
  end
end

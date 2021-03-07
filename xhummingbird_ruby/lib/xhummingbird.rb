# frozen_string_literal: true

require 'singleton'
require 'ffi-rzmq'

require_relative "xhummingbird/version"
require_relative "xhummingbird/client"
require_relative "xhummingbird/protos/event_pb"

module Xhummingbird
  class Error < StandardError; end

  def self.send(**args)
    if enabled?
      event = Event.new(**args)
      message = Event.encode(event)
      Client.instance.send(message)
    end
  rescue
    raise Error
  end

  def self.enabled?
    Client.instance.enabled?
  end
end

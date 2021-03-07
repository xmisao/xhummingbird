# frozen_string_literal: true

require_relative "lib/xhummingbird/version"

Gem::Specification.new do |spec|
  spec.name          = "xhummingbird"
  spec.version       = Xhummingbird::VERSION
  spec.authors       = ["xmisao"]
  spec.email         = ["mail@xmisao.com"]

  spec.summary       = "Ruby client for xHummingbird"
  spec.description   = "The Ruby client for xHummingbird application event tracker"
  spec.homepage      = "https://github.com/xmisao/xhummingbird/xhummignbird_ruby/"
  spec.license       = "MIT"
  spec.required_ruby_version = Gem::Requirement.new(">= 2.3.0")

  # spec.metadata["allowed_push_host"] = "TODO: Set to 'http://mygemserver.com'"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
  # spec.metadata["changelog_uri"] = "TODO: Put your gem's CHANGELOG.md URL here."

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir.chdir(File.expand_path(__dir__)) do
    `git ls-files -z`.split("\x0").reject { |f| f.match(%r{\A(?:test|spec|features)/}) }
  end
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  # Uncomment to register a new dependency of your gem
  # spec.add_dependency "example-gem", "~> 1.0"
  spec.add_dependency "ffi-rzmq"
  spec.add_dependency "google-protobuf"

  # For more information and examples about making a new gem, checkout our
  # guide at: https://bundler.io/guides/creating_gem.html
end

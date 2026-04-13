# Homebrew formula for ccswitch.
#
# This file belongs in a tap repo, e.g.:
#   https://github.com/gautamkumar7atgit/homebrew-ccswitch
#
# Install command:
#   brew tap gautamkumar7atgit/ccswitch
#   brew install ccswitch
#
# To update after a new release:
#   1. Download the new universal binary and run:
#        shasum -a 256 ccswitch
#   2. Update `url` and `sha256` below.
#   3. Commit and push to the tap repo.

class Ccswitch < Formula
  desc "Switch between multiple Claude Code OAuth accounts on macOS"
  homepage "https://github.com/gautamkumar7atgit/ClaudeCodeSwitch"
  version "0.1.1"

  # Universal binary (arm64 + x86_64)
  url "https://github.com/gautamkumar7atgit/ClaudeCodeSwitch/releases/download/v#{version}/ccswitch"
  sha256 "ba8e4970c230f77a6434d180f67c0dad78f0873366f4756d7ae6f6123802af50"

  # No build step — we distribute a pre-built universal binary
  bottle :unneeded

  def install
    bin.install "ccswitch"
  end

  def caveats
    <<~EOS
      To set up ccswitch for the first time, run:
        ccswitch init

      This will import your existing Claude Code credentials and start
      the background sync daemon via launchd.
    EOS
  end

  test do
    assert_match "ccswitch", shell_output("#{bin}/ccswitch --version")
  end
end

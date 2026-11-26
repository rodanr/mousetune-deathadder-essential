# Homebrew cask for MouseTune, published in the tap rodanr/homebrew-tap as
# Casks/mousetune.rb. Install: brew install --cask rodanr/tap/mousetune
#
# The tap's daily `brew bump` workflow keeps version and sha256 current via
# livecheck; this copy is the source of truth for everything else.
cask "mousetune" do
  version "0.1.0"
  sha256 :no_check # replaced with the DMG's SHA-256 (from SHA256SUMS.txt) when published

  url "https://github.com/rodanr/mousetune-deathadder-essential/releases/download/v#{version}/MouseTune-#{version}-macos-universal.dmg"
  name "MouseTune"
  desc "Set DPI, polling rate and lighting on the Razer DeathAdder Essential"
  homepage "https://github.com/rodanr/mousetune-deathadder-essential"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :ventura"

  app "MouseTune.app"

  # MouseTune is ad-hoc signed, not notarized, so Gatekeeper would block the
  # first launch. Remove this once releases are signed and notarized.
  postflight_steps do
    run "/usr/bin/xattr", args: ["-dr", "com.apple.quarantine", "{{appdir}}/MouseTune.app"]
  end

  zap trash: "~/Library/Application Support/io.github.rodanr.MouseTune"

  caveats <<~EOS
    MouseTune needs Input Monitoring permission to talk to the mouse.
    Allow it when asked, or in System Settings > Privacy & Security > Input Monitoring.
  EOS
end

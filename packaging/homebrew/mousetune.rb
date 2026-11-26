# Homebrew formula for MouseTune, intended for the tap rodanr/homebrew-tap.
#
# Install:  brew install rodanr/tap/mousetune
#
# After tagging a release, set `sha256` to the source tarball checksum (the
# release workflow prints it) and copy this file to the tap's Formula/ folder.
class Mousetune < Formula
  desc "Set DPI, polling rate and lighting on the Razer DeathAdder Essential"
  homepage "https://github.com/rodanr/mousetune-deathadder-essential"
  url "https://github.com/rodanr/mousetune-deathadder-essential/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000" # placeholder until the first release
  license "GPL-3.0-or-later"
  head "https://github.com/rodanr/mousetune-deathadder-essential.git", branch: "main"

  depends_on "rust" => :build
  depends_on :macos

  def install
    # Build the binary into libexec, then wrap it into an app bundle.
    system "cargo", "install", *std_cargo_args(path: "app", root: libexec)

    contents = prefix/"MouseTune.app/Contents"
    (contents/"MacOS").mkpath
    (contents/"Resources").mkpath
    cp libexec/"bin/mousetune", contents/"MacOS/MouseTune"
    cp "assets/AppIcon.icns", contents/"Resources/AppIcon.icns"
    (contents/"Info.plist").write(
      (buildpath/"packaging/macos/Info.plist").read.gsub("@VERSION@", version.to_s),
    )

    (bin/"mousetune").write <<~SH
      #!/bin/bash
      exec open "#{opt_prefix}/MouseTune.app"
    SH
    chmod 0555, bin/"mousetune"

    # No paid Apple Developer account: ad-hoc signature only.
    system "codesign", "--force", "--deep", "--sign", "-", prefix/"MouseTune.app"
  end

  def caveats
    <<~EOS
      MouseTune.app was installed to:
        #{opt_prefix}/MouseTune.app

      Start it with:
        mousetune

      To make it appear in Launchpad and Spotlight, copy it to /Applications:
        cp -R "#{opt_prefix}/MouseTune.app" /Applications/

      (A symlink also works: ln -sf "#{opt_prefix}/MouseTune.app" /Applications/MouseTune.app)

      On first launch macOS asks for Input Monitoring access. If it does not,
      open System Settings > Privacy & Security > Input Monitoring and enable
      MouseTune. The app is ad-hoc signed, so macOS may ask again after an upgrade.
    EOS
  end

  test do
    assert_path_exists prefix/"MouseTune.app/Contents/MacOS/MouseTune"
    assert_match "io.github.rodanr.mousetune", (prefix/"MouseTune.app/Contents/Info.plist").read
    system "codesign", "--verify", "--deep", prefix/"MouseTune.app"
  end
end

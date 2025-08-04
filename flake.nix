{
  description = "R-Hyprconfig - A modern TUI for managing Hyprland configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = inputs@{ self, nixpkgs, flake-utils, rust-overlay, devenv, ... }:
    let
      # Per-system outputs
      systemOutputs = flake-utils.lib.eachDefaultSystem (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          # Rust toolchain
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          };

          # Build dependencies
          buildInputs = with pkgs; [
            # Core build dependencies
            pkg-config
            openssl
            
            # For crossterm/terminal handling
            libiconv
            
            # Development tools
            git
            just           # Task automation
            bacon          # Rust development tool
            cargo-watch    # Auto-rebuild on changes
            cargo-edit     # Cargo extensions for editing Cargo.toml
            cargo-audit    # Security auditing
            cargo-deny     # Dependency analysis
            cargo-outdated # Check for outdated dependencies
            
            # Optional: for better terminal experience
            ncurses
          ];

          # Native build inputs (platform-specific)
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          # The main r-hyprconfig package
          r-hyprconfig = pkgs.rustPlatform.buildRustPackage {
            pname = "r-hyprconfig";
            version = "1.2.0";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            inherit nativeBuildInputs buildInputs;

            # Skip tests during build (they require Hyprland to be running)
            doCheck = false;

            meta = with pkgs.lib; {
              description = "A modern TUI for managing Hyprland configuration";
              homepage = "https://github.com/olafkfreund/r-hyprconfig";
              license = licenses.mit;
              maintainers = [ ];
              platforms = platforms.linux;
            };
          };

        in
        {
          # Development environment
          devShells.default = pkgs.mkShell {
            inherit buildInputs nativeBuildInputs;
            
            # Environment variables
            RUST_LOG = "debug";
            RUST_BACKTRACE = "1";
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            CARGO_HOME = ".cargo";
            RUSTUP_HOME = ".rustup";
            
            # Shell hook
            shellHook = ''
              echo "🦀 Welcome to R-Hyprconfig development environment!"
              echo "📦 Rust toolchain: $(rustc --version)"
              echo "🔧 Available tools:"
              echo "  - cargo build          # Build the project"
              echo "  - cargo run            # Run the application" 
              echo "  - cargo test           # Run tests"
              echo "  - cargo clippy         # Lint the code"
              echo "  - cargo fmt            # Format the code"
              echo "  - cargo watch -x run   # Auto-rebuild and run"
              echo "  - just <command>       # Task automation"
              echo ""
              echo "🏗️  Nix commands:"
              echo "  - nix build .#r-hyprconfig  # Build with Nix"
              echo "  - nix run .#r-hyprconfig    # Run with Nix"
              echo ""
              
              # Create necessary directories
              mkdir -p .cargo
              
              # Set up cargo config for better performance
              cat > .cargo/config.toml << 'EOF'
[build]
jobs = 8

[profile.dev]
debug = true
incremental = true

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
EOF

              echo "📁 Development environment ready!"
              echo "🚀 Run 'cargo run -- --debug' to start the application in debug mode"
            '';
          };

          # Package outputs
          packages = {
            default = r-hyprconfig;
            r-hyprconfig = r-hyprconfig;
          };

          # Application to run
          apps = {
            default = {
              type = "app";
              program = "${r-hyprconfig}/bin/r-hyprconfig";
              meta = {
                description = "A modern TUI for managing Hyprland configuration";
                license = pkgs.lib.licenses.mit;
              };
            };
            r-hyprconfig = {
              type = "app";
              program = "${r-hyprconfig}/bin/r-hyprconfig";
              meta = {
                description = "A modern TUI for managing Hyprland configuration";
                license = pkgs.lib.licenses.mit;
              };
            };
            dev = {
              type = "app";
              program = "${pkgs.writeShellScript "dev-r-hyprconfig" ''
                cd ${./.}
                exec ${rustToolchain}/bin/cargo run -- "$@"
              ''}";
              meta = {
                description = "Development version of r-hyprconfig";
                license = pkgs.lib.licenses.mit;
              };
            };
          };

          # Formatter for the flake
          formatter = pkgs.nixpkgs-fmt;
        });

      # Create the NixOS module
      nixosModule = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.programs.r-hyprconfig;
          # Get the package from the current system
          r-hyprconfig = systemOutputs.packages.${pkgs.stdenv.hostPlatform.system}.r-hyprconfig;
        in
        {
          options.programs.r-hyprconfig = {
            enable = mkEnableOption "r-hyprconfig";
            package = mkOption {
              type = types.package;
              default = r-hyprconfig;
              description = "The r-hyprconfig package to use";
            };
          };

          config = mkIf cfg.enable {
            environment.systemPackages = [ cfg.package ];
          };
        };

      # Create the Home Manager module
      homeManagerModule = { config, lib, pkgs, ... }:
        with lib;
        let
          cfg = config.programs.r-hyprconfig;
          # Get the package from the current system
          r-hyprconfig = systemOutputs.packages.${pkgs.stdenv.hostPlatform.system}.r-hyprconfig;
        in
        {
          options.programs.r-hyprconfig = {
            enable = mkEnableOption "r-hyprconfig";
            package = mkOption {
              type = types.package;
              default = r-hyprconfig;
              description = "The r-hyprconfig package to use";
            };
            settings = mkOption {
              type = types.attrs;
              default = { };
              description = "Configuration options for r-hyprconfig";
            };
          };

          config = mkIf cfg.enable {
            home.packages = [ cfg.package ];
            
            # Optional: Generate config file from settings
            xdg.configFile."r-hyprconfig/config.toml" = mkIf (cfg.settings != { }) {
              text = lib.generators.toTOML { } cfg.settings;
            };
          };
        };

    in
    systemOutputs // {
      # Top-level modules for easy access
      nixosModules.default = nixosModule;
      homeManagerModules.default = homeManagerModule;
    };
}
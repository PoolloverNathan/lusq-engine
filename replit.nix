{ pkgs }: {
	deps = [
		pkgs.screen
  pkgs.vim
  pkgs.unzip
  pkgs.adoptopenjdk-openj9-bin-8
  pkgs.rustup
  pkgs.kotlin
		pkgs.gradle
		pkgs.maven
		pkgs.kotlin-language-server
	];
}
defmodule Ibento.Client.MixProject do
  use Mix.Project

  def project do
    [
      app: :ibento_client,
      version: "0.1.0",
      elixir: "~> 1.8",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      aliases: aliases()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger, :grpcbox],
      mod: {Ibento.Client.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:gpb, "4.7.2"},
      {:grpcbox, github: "potatosalad/grpcbox", branch: "wip", runtime: false},
    ]
  end

  defp aliases() do
    [
      test: ["test --no-start"]
    ]
  end
end

defmodule Ibento.ClientTest do
  use ExUnit.Case
  doctest Ibento.Client

  test "greets the world" do
    assert Ibento.Client.hello() == :world
  end
end

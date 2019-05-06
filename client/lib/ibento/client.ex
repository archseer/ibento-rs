defmodule Ibento.Client do
  @moduledoc """
  """

  @spec subscribe(%{topics: [String.t()]}) :: {:ok, stream :: term()} | {:error, reason :: term()}
  def subscribe(request) do
    :ibento_ibento_client.subscribe(request)
  end
end

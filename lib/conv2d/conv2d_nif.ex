defmodule Conv2dNif do
	use Rustler, otp_app: :conv2d, crate: :Conv2dNif

	def mult(a) do
		:ok = mult_n(a)
		l = receive do
			l -> l
		end
		case l do
			{:error, message} -> raise message
			_ -> l
		end
	end

	def mult_n(_a), do: exit(:nif_not_loaded)
end
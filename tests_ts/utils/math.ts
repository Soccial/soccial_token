/**
 * Converts a token amount to its raw integer representation (e.g., 1 token → 1_000_000_000 for 9 decimals).
 * Always returns a `bigint`.
 *
 * @param value The amount as number, string, or bigint.
 * @param decimals Number of decimals (default: 9)
 */
export function toTokenAmount(value: number | string | bigint, decimals: number = 9): bigint {
  const multiplier = BigInt(10 ** decimals);

  if (typeof value === "bigint") return value * multiplier;
  if (typeof value === "number") return BigInt(Math.floor(value * 10 ** decimals));
  if (typeof value === "string") return BigInt(value) * multiplier;

  throw new Error("Unsupported value type for toTokenAmount");
}

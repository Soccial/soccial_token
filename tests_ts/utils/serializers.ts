import { Buffer } from "buffer";

/**
 * Serializes arguments to a Buffer.
 * 
 * Note:
 * - This simple implementation encodes the arguments as a JSON array.
 * - If your program expects a different format (e.g., Borsh), adjust accordingly.
 *
 * @param args - An array or object representing the instruction arguments.
 * @returns A Buffer containing the serialized arguments.
 */
export function serializeArgs(args: any): Buffer {
  return Buffer.from(JSON.stringify(args), "utf-8");
}

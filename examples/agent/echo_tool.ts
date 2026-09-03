export async function echo(input: { text?: string }) {
  return { text: String(input?.text ?? "") };
}

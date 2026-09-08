/**
 * Beejs Runtime Type Definitions
 * Version: 1.0.0
 *
 * Comprehensive TypeScript declarations for Beejs runtime built-in APIs,
 * global objects, and native agentic AI modules.
 */

declare module "bee:ai" {
  /**
   * High-performance, zero-copy n-dimensional Tensor backed by TypedArray.
   */
  export class Tensor {
    readonly shape: readonly number[];
    readonly strides: readonly number[];
    readonly data: Float32Array;
    readonly length: number;

    constructor(data: ArrayLike<number> | Float32Array, shape?: number[]);

    /**
     * Creates a tensor filled with zeros.
     */
    static zeros(shape: number[]): Tensor;

    /**
     * Creates a tensor filled with ones.
     */
    static ones(shape: number[]): Tensor;

    /**
     * Matrix multiplication with another 2D tensor.
     */
    matmul(other: Tensor): Tensor;

    /**
     * Vector dot product.
     */
    dot(other: Tensor): number;

    /**
     * Computes L2 Frobenius norm.
     */
    norm(): number;

    /**
     * Applies softmax normalization along the last dimension.
     */
    softmax(): Tensor;

    /**
     * Computes cosine similarity between two vectors.
     */
    cosineSimilarity(other: Tensor): number;

    /**
     * Element-wise addition.
     */
    add(other: Tensor | number): Tensor;

    /**
     * Element-wise subtraction.
     */
    sub(other: Tensor | number): Tensor;

    /**
     * Element-wise multiplication.
     */
    mul(other: Tensor | number): Tensor;

    /**
     * Element-wise division.
     */
    div(other: Tensor | number): Tensor;

    /**
     * Flattens tensor data into a standard JavaScript array.
     */
    toArray(): number[];

    /**
     * Slices tensor along first dimension.
     */
    slice(start: number, end?: number): Tensor;
  }

  export interface LLMGenerateOptions {
    prompt: string;
    maxTokens?: number;
    temperature?: number;
    topP?: number;
    stop?: string[];
  }

  export interface LLMResponse {
    text: string;
    tokensUsed: number;
    finishReason: "stop" | "length" | "timeout";
  }

  /**
   * Local streaming LLM inference engine.
   */
  export class LLM {
    readonly modelPath: string;

    constructor(modelPath?: string);

    /**
     * Loads a local weights file or GGUF/ONNX model into memory.
     */
    static load(modelPath: string): Promise<LLM>;

    /**
     * Generates a complete response synchronously or asynchronously.
     */
    generate(options: LLMGenerateOptions): Promise<LLMResponse>;

    /**
     * Streams generated tokens sequentially.
     */
    generateStream(
      options: LLMGenerateOptions
    ): AsyncIterableIterator<string>;

    /**
     * Computes dense vector embeddings for input text.
     */
    embed(text: string): Promise<Tensor>;
  }

  export interface AgentTool {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
    execute: (args: Record<string, unknown>) => Promise<unknown> | unknown;
  }

  /**
   * Deterministic pipeline orchestrator for Agent tools.
   */
  export class AgentPipeline {
    readonly tools: Map<string, AgentTool>;

    constructor();

    /**
     * Registers an executable tool into the pipeline.
     */
    registerTool(tool: AgentTool): this;

    /**
     * Executes a tool invocation plan deterministically.
     */
    execute(plan: { tool: string; args: Record<string, unknown> }): Promise<unknown>;
  }
}

declare module "bee:bench" {
  export interface BenchOptions {
    warmup?: number;
    iterations?: number;
  }

  export function bench(name: string, fn: () => void | Promise<void>, options?: BenchOptions): void;
}

/**
 * Beejs Runtime Global Namespace
 */
declare namespace bee {
  export const version: string;

  /**
   * Registers a microbenchmark.
   */
  export function bench(name: string, fn: () => void | Promise<void>): void;

  /**
   * Active sandbox permissions.
   */
  export const permissions: {
    readonly denyFs: boolean;
    readonly denyNet: boolean;
    readonly denyEnv: boolean;
    readonly denyRun: boolean;
    readonly isSandbox: boolean;
  };
}

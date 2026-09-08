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

declare module "bee:db" {
  export interface DatabaseOptions {
    readonly?: boolean;
  }

  export interface RunResult {
    changes: number;
    lastInsertRowid: number;
  }

  export interface PreparedStatement {
    all<T = Record<string, unknown>>(...params: unknown[]): T[];
    get<T = Record<string, unknown>>(...params: unknown[]): T | null;
    run(...params: unknown[]): RunResult;
  }

  export class Database {
    readonly filename: string;
    readonly readonly: boolean;

    constructor(filename?: string, options?: DatabaseOptions);
    exec(sql: string): this;
    prepare(sql: string): PreparedStatement;
    query<T = Record<string, unknown>>(sql: string, ...params: unknown[]): T[];
    run(sql: string, ...params: unknown[]): RunResult;
    transaction<T>(fn: () => T): T;
    close(): void;
  }

  export { VectorDB } from "bee:vector";
}

declare module "bee:sqlite" {
  export * from "bee:db";
}

declare module "bee:vector" {
  export type VectorMetric = "cosine" | "euclidean" | "dot";

  export interface VectorDBOptions {
    dimensions?: number;
    metric?: VectorMetric;
  }

  export interface SearchOptions {
    topK?: number;
    threshold?: number;
    filter?: (metadata: Record<string, unknown>) => boolean;
  }

  export interface SearchResult {
    id: string;
    score: number;
    metadata: Record<string, unknown>;
    vector: Float32Array;
  }

  export class VectorDB {
    dimensions: number;
    metric: VectorMetric;
    readonly size: number;

    constructor(options?: VectorDBOptions);
    insert(
      id: string | number,
      vector: number[] | Float32Array | { data: Float32Array },
      metadata?: Record<string, unknown>
    ): this;
    delete(id: string | number): boolean;
    get(id: string | number): { id: string; vector: Float32Array; metadata: Record<string, unknown> } | null;
    search(
      queryVector: number[] | Float32Array | { data: Float32Array },
      options?: SearchOptions
    ): SearchResult[];
    toJSON(): Record<string, unknown>;
    static fromJSON(data: Record<string, unknown>): VectorDB;
  }

  export function cosineSimilarity(a: Float32Array | number[], b: Float32Array | number[]): number;
  export function euclideanDistance(a: Float32Array | number[], b: Float32Array | number[]): number;
  export function dotProduct(a: Float32Array | number[], b: Float32Array | number[]): number;
}

declare module "bee:std/dotenv" {
  export interface DotenvConfigOptions {
    path?: string;
    override?: boolean;
  }

  export interface DotenvResult {
    parsed: Record<string, string>;
    error?: Error;
  }

  export function parse(src: string): Record<string, string>;
  export function config(options?: DotenvConfigOptions): DotenvResult;
}

declare module "bee:std/cli" {
  export const colors: {
    reset: (s: string) => string;
    bold: (s: string) => string;
    dim: (s: string) => string;
    red: (s: string) => string;
    green: (s: string) => string;
    yellow: (s: string) => string;
    blue: (s: string) => string;
    magenta: (s: string) => string;
    cyan: (s: string) => string;
    gray: (s: string) => string;
    bgRed: (s: string) => string;
    bgGreen: (s: string) => string;
    [key: string]: (s: string) => string;
  };

  export function table(data: Record<string, unknown>[], columns?: string[]): string;

  export interface ProgressBar {
    tick(delta?: number): void;
    render(): void;
    complete(): void;
  }

  export function progressBar(options?: { total?: number; width?: number }): ProgressBar;
  export function prompt(question: string, defaultValue?: string): Promise<string>;
}

declare module "bee:std/fs" {
  export interface FileEntry {
    path: string;
    name: string;
    isFile: boolean;
    isDirectory: boolean;
    size: number;
  }

  export interface WalkDirOptions {
    maxDepth?: number;
    exts?: string[];
  }

  export function walkDir(dir?: string, options?: WalkDirOptions): FileEntry[];
  export function copyDir(src: string, dest: string): void;
  export function emptyDir(dir: string): void;
  export function ensureDir(dir: string): void;
  export function ensureFile(file: string): void;
}

declare module "bee:std/crypto" {
  export interface JWTSignOptions {
    expiresIn?: number | string;
    algorithm?: "HS256";
  }

  export const jwt: {
    sign(payload: Record<string, unknown>, secret: string, options?: JWTSignOptions): string;
    verify<T = Record<string, unknown>>(token: string, secret: string): T;
  };

  export function hash(algorithm: "sha256" | "sha512" | "md5", data: string | Uint8Array, encoding?: "hex" | "base64"): string;
  export function uuid(): string;
  export function uuidv7(): string;
}

declare module "bee:std/assert" {
  export function assert(condition: unknown, message?: string): asserts condition;
  export function assertEquals<T>(actual: unknown, expected: T, message?: string): asserts actual is T;
  export function assertNotEquals(actual: unknown, expected: unknown, message?: string): void;
  export function assertThrows(fn: () => unknown, expectedError?: string | RegExp, message?: string): void;
}

declare module "bee:std" {
  export * as dotenv from "bee:std/dotenv";
  export * as cli from "bee:std/cli";
  export * as fs from "bee:std/fs";
  export * as crypto from "bee:std/crypto";
  export { assert, assertEquals, assertNotEquals, assertThrows } from "bee:std/assert";
}

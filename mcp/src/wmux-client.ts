import net from "net";

const PIPE_PATH = "\\\\.\\pipe\\wmux";

interface WmuxResponse {
  id: string;
  ok: boolean;
  result?: unknown;
  error?: { code: string; message: string };
}

export class WmuxClient {
  private socket: net.Socket | null = null;
  private buffer = "";
  private pending = new Map<
    string,
    { resolve: (value: WmuxResponse) => void; reject: (err: Error) => void }
  >();
  private nextId = 1;

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.socket = net.connect(PIPE_PATH, () => resolve());
      this.socket.on("error", (err) => {
        for (const [, { reject: rej }] of this.pending) {
          rej(err);
        }
        this.pending.clear();
        reject(err);
      });
      this.socket.on("data", (data) => this.onData(data));
      this.socket.on("close", () => {
        this.socket = null;
      });
    });
  }

  private onData(data: Buffer): void {
    this.buffer += data.toString();
    const lines = this.buffer.split("\n");
    this.buffer = lines.pop() ?? "";

    for (const line of lines) {
      if (!line.trim()) continue;
      try {
        const response: WmuxResponse = JSON.parse(line);
        const pending = this.pending.get(response.id);
        if (pending) {
          this.pending.delete(response.id);
          pending.resolve(response);
        }
      } catch {
        // Ignore malformed responses
      }
    }
  }

  async call(method: string, params?: Record<string, unknown>): Promise<unknown> {
    if (!this.socket) {
      await this.connect();
    }

    const id = String(this.nextId++);
    const request = JSON.stringify({ id, method, params: params ?? {} });

    return new Promise<unknown>((resolve, reject) => {
      this.pending.set(id, {
        resolve: (resp) => {
          if (resp.ok) {
            resolve(resp.result);
          } else {
            reject(
              new Error(
                `wmux error [${resp.error?.code}]: ${resp.error?.message}`
              )
            );
          }
        },
        reject,
      });

      this.socket!.write(request + "\n", (err) => {
        if (err) {
          this.pending.delete(id);
          reject(err);
        }
      });

      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error("wmux request timed out"));
        }
      }, 10000);
    });
  }

  disconnect(): void {
    this.socket?.destroy();
    this.socket = null;
  }

  get connected(): boolean {
    return this.socket !== null && !this.socket.destroyed;
  }
}

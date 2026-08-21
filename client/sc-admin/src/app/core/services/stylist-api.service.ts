import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable, Observer } from 'rxjs';
import { AuthStore } from '../../store/auth.store';

/** SSE events emitted by ai-service (see server/ai-service/DESIGN.md). */
export type StylistEvent =
  | { kind: 'token'; text: string }
  | { kind: 'products'; ids: string[] }
  | { kind: 'ping' }
  | { kind: 'error'; code: string; message: string }
  | { kind: 'done' };

interface CreateConversationResponse {
  conversation_id: string;
}

@Injectable({ providedIn: 'root' })
export class StylistApiService {
  private http = inject(HttpClient);
  private authStore = inject(AuthStore);
  private base = '/api/stylist';

  createConversation(): Observable<CreateConversationResponse> {
    return this.http.post<CreateConversationResponse>(`${this.base}/conversations`, {});
  }

  deleteConversation(id: string): Observable<void> {
    return this.http.delete<void>(`${this.base}/conversations/${id}`);
  }

  /** Stream a chat reply. The returned observable emits decoded SSE events
   *  in order; subscribers should unsubscribe to abort the stream. */
  streamChat(conversationId: string, message: string): Observable<StylistEvent> {
    return new Observable<StylistEvent>((observer: Observer<StylistEvent>) => {
      const controller = new AbortController();
      const token = this.authStore.accessToken();

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        Accept: 'text/event-stream',
      };
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }

      const run = async () => {
        let response: Response;
        try {
          response = await fetch(`${this.base}/chat`, {
            method: 'POST',
            headers,
            body: JSON.stringify({ conversation_id: conversationId, message }),
            signal: controller.signal,
          });
        } catch (err) {
          observer.error(err);
          return;
        }

        if (!response.ok) {
          observer.next({
            kind: 'error',
            code: response.status === 401 ? 'unauthorized' : 'http_error',
            message: `${response.status} ${response.statusText}`,
          });
          observer.next({ kind: 'done' });
          observer.complete();
          return;
        }
        if (!response.body) {
          observer.next({ kind: 'error', code: 'no_body', message: 'no response body' });
          observer.next({ kind: 'done' });
          observer.complete();
          return;
        }

        const reader = response.body.getReader();
        const decoder = new TextDecoder('utf-8');
        let buffer = '';

        const dispatch = (block: string) => {
          let eventName = 'message';
          const dataLines: string[] = [];
          for (const rawLine of block.split('\n')) {
            const line = rawLine.replace(/\r$/, '');
            if (!line || line.startsWith(':')) continue; // skip comments / blanks
            if (line.startsWith('event:')) {
              eventName = line.slice('event:'.length).trim();
            } else if (line.startsWith('data:')) {
              dataLines.push(line.slice('data:'.length).trim());
            }
          }
          const data = dataLines.join('\n') || '{}';
          let parsed: Record<string, unknown>;
          try {
            parsed = JSON.parse(data);
          } catch {
            parsed = {};
          }
          const ev = parseStylistEvent(eventName, parsed);
          if (ev) observer.next(ev);
        };

        try {
          while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            // sse-starlette emits CRLF (\r\n\r\n) between blocks; normalise
            // to LF so a single `\n\n` split works regardless of upstream.
            buffer += decoder.decode(value, { stream: true }).replace(/\r\n/g, '\n');

            let sep = buffer.indexOf('\n\n');
            while (sep !== -1) {
              const block = buffer.slice(0, sep);
              buffer = buffer.slice(sep + 2);
              dispatch(block);
              sep = buffer.indexOf('\n\n');
            }
          }
          if (buffer.trim().length > 0) {
            dispatch(buffer);
          }
        } catch (err) {
          if (!controller.signal.aborted) {
            observer.error(err);
            return;
          }
        }

        observer.complete();
      };

      void run();

      return () => {
        controller.abort();
      };
    });
  }
}

function parseStylistEvent(name: string, payload: Record<string, unknown>): StylistEvent | null {
  switch (name) {
    case 'token':
      return { kind: 'token', text: typeof payload['text'] === 'string' ? payload['text'] : '' };
    case 'products': {
      const ids = Array.isArray(payload['ids']) ? (payload['ids'] as unknown[]) : [];
      return {
        kind: 'products',
        ids: ids.filter((x): x is string => typeof x === 'string'),
      };
    }
    case 'ping':
      return { kind: 'ping' };
    case 'error':
      return {
        kind: 'error',
        code: typeof payload['code'] === 'string' ? payload['code'] : 'unknown',
        message: typeof payload['message'] === 'string' ? payload['message'] : '',
      };
    case 'done':
      return { kind: 'done' };
    default:
      return null;
  }
}

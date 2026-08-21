import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  OnDestroy,
  ViewChild,
  computed,
  inject,
  signal,
} from '@angular/core';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';
import { ButtonModule } from 'primeng/button';
import { InputTextModule } from 'primeng/inputtext';
import { TextareaModule } from 'primeng/textarea';
import { MessageModule } from 'primeng/message';
import { Subscription, firstValueFrom } from 'rxjs';
import { ProductApiService } from '../../core/services/product-api.service';
import { AdminProductDTO } from '../../core/models/product.model';
import { StylistApiService, StylistEvent } from '../../core/services/stylist-api.service';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  text: string;
  /** When set, render product cards after the message body. */
  productIds?: string[];
  /** When set, render an inline error chip. */
  error?: { code: string; message: string };
  streaming?: boolean;
}

@Component({
  selector: 'app-stylist-chat',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    FormsModule,
    RouterLink,
    TranslateModule,
    ButtonModule,
    InputTextModule,
    TextareaModule,
    MessageModule,
  ],
  templateUrl: './stylist-chat.component.html',
  styleUrl: './stylist-chat.component.scss',
})
export class StylistChatComponent implements OnDestroy {
  private api = inject(StylistApiService);
  private products = inject(ProductApiService);

  @ViewChild('scrollAnchor') scrollAnchor?: ElementRef<HTMLDivElement>;

  conversationId = signal<string | null>(null);
  messages = signal<ChatMessage[]>([]);
  productsCache = signal<Record<string, AdminProductDTO | 'loading' | 'error'>>({});
  draft = signal('');
  streaming = signal(false);
  startError = signal<string | null>(null);

  canSend = computed(
    () => !this.streaming() && this.draft().trim().length > 0 && this.conversationId() !== null,
  );

  private streamSub: Subscription | null = null;
  private assistantId: string | null = null;

  ngOnDestroy(): void {
    this.streamSub?.unsubscribe();
  }

  async start(): Promise<void> {
    this.startError.set(null);
    try {
      const { conversation_id } = await firstValueFrom(this.api.createConversation());
      this.conversationId.set(conversation_id);
      this.messages.set([]);
    } catch (e) {
      this.startError.set(this.errMessage(e));
    }
  }

  async clear(): Promise<void> {
    const id = this.conversationId();
    this.streamSub?.unsubscribe();
    this.streaming.set(false);
    this.messages.set([]);
    if (id) {
      try {
        await firstValueFrom(this.api.deleteConversation(id));
      } catch {
        // Idempotent on the server — swallow.
      }
    }
    this.conversationId.set(null);
  }

  send(): void {
    if (!this.canSend()) return;
    const id = this.conversationId();
    if (!id) return;
    const text = this.draft().trim();
    this.draft.set('');

    const userMsg: ChatMessage = { id: this.uid(), role: 'user', text };
    const assistant: ChatMessage = {
      id: this.uid(),
      role: 'assistant',
      text: '',
      streaming: true,
    };
    this.assistantId = assistant.id;
    this.messages.update(m => [...m, userMsg, assistant]);
    this.streaming.set(true);
    this.queueScroll();

    this.streamSub?.unsubscribe();
    this.streamSub = this.api.streamChat(id, text).subscribe({
      next: ev => this.onEvent(ev),
      error: err => this.finishWithError(err),
      complete: () => this.finishStream(),
    });
  }

  abort(): void {
    this.streamSub?.unsubscribe();
    this.streamSub = null;
    this.finishStream();
  }

  trackByMessage = (_: number, m: ChatMessage) => m.id;
  trackById = (_: number, id: string) => id;

  productFor(id: string): AdminProductDTO | 'loading' | 'error' | undefined {
    return this.productsCache()[id];
  }

  productImageUrl(p: AdminProductDTO): string | null {
    return p.preview_url?.medium ?? p.image_urls[0]?.medium ?? null;
  }

  private onEvent(ev: StylistEvent): void {
    if (ev.kind === 'token') {
      this.appendToAssistant(ev.text);
      return;
    }
    if (ev.kind === 'products') {
      this.attachProducts(ev.ids);
      void this.preloadCards(ev.ids);
      return;
    }
    if (ev.kind === 'error') {
      this.attachError(ev.code, ev.message);
      return;
    }
    if (ev.kind === 'done') {
      this.finishStream();
    }
    // 'ping' — no-op
  }

  private appendToAssistant(text: string): void {
    if (!text || !this.assistantId) return;
    const id = this.assistantId;
    this.messages.update(list =>
      list.map(m => (m.id === id ? { ...m, text: m.text + text } : m)),
    );
    this.queueScroll();
  }

  private attachProducts(ids: string[]): void {
    if (!this.assistantId || ids.length === 0) return;
    const id = this.assistantId;
    this.messages.update(list =>
      list.map(m => (m.id === id ? { ...m, productIds: ids } : m)),
    );
    this.queueScroll();
  }

  private attachError(code: string, message: string): void {
    if (!this.assistantId) return;
    const id = this.assistantId;
    this.messages.update(list =>
      list.map(m => (m.id === id ? { ...m, error: { code, message } } : m)),
    );
  }

  private finishStream(): void {
    this.streaming.set(false);
    if (this.assistantId) {
      const id = this.assistantId;
      this.messages.update(list =>
        list.map(m => (m.id === id ? { ...m, streaming: false } : m)),
      );
    }
    this.assistantId = null;
    this.queueScroll();
  }

  private finishWithError(err: unknown): void {
    this.attachError('transport', this.errMessage(err));
    this.finishStream();
  }

  private async preloadCards(ids: string[]): Promise<void> {
    const cache = this.productsCache();
    const need = ids.filter(id => !(id in cache));
    if (need.length === 0) return;

    this.productsCache.update(c => {
      const next = { ...c };
      for (const id of need) next[id] = 'loading';
      return next;
    });

    await Promise.all(
      need.map(async id => {
        try {
          const res = await firstValueFrom(this.products.getProduct(id));
          const data = res.data;
          this.productsCache.update(c => ({ ...c, [id]: data ?? 'error' }));
        } catch {
          this.productsCache.update(c => ({ ...c, [id]: 'error' }));
        }
      }),
    );
  }

  private queueScroll(): void {
    queueMicrotask(() => {
      this.scrollAnchor?.nativeElement?.scrollIntoView({ behavior: 'smooth', block: 'end' });
    });
  }

  private uid(): string {
    return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
  }

  private errMessage(e: unknown): string {
    if (e instanceof Error) return e.message;
    return String(e);
  }
}

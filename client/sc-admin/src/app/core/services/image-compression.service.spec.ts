import { TestBed } from '@angular/core/testing';
import { describe, expect, it, beforeEach } from 'vitest';
import { ImageCompressionService } from './image-compression.service';

describe('ImageCompressionService', () => {
  let service: ImageCompressionService;

  beforeEach(() => {
    TestBed.configureTestingModule({ providers: [ImageCompressionService] });
    service = TestBed.inject(ImageCompressionService);
  });

  it('returns non-image files unchanged', async () => {
    const pdf = new File(['x'], 'a.pdf', { type: 'application/pdf' });
    const result = await service.compress(pdf);
    expect(result).toBe(pdf);
  });

  it('returns the input file as-is when content-type is text', async () => {
    const txt = new File(['hello'], 'note.txt', { type: 'text/plain' });
    const out = await service.compress(txt);
    expect(out).toBe(txt);
    expect(out.name).toBe('note.txt');
  });

  it('compressMany preserves array length', async () => {
    const files = [
      new File(['1'], '1.pdf', { type: 'application/pdf' }),
      new File(['2'], '2.txt', { type: 'text/plain' }),
    ];
    const out = await service.compressMany(files);
    expect(out).toHaveLength(2);
  });
});

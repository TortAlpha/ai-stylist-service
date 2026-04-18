import { Injectable } from '@angular/core';
import imageCompression from 'browser-image-compression';

/**
 * Compresses product photos in the browser before upload to keep payload small
 * (target ~2 MB, longest edge 2048 px, WebP q=0.85). EXIF is auto-rotated and
 * stripped by the library.
 */
@Injectable({ providedIn: 'root' })
export class ImageCompressionService {
  private readonly options: Parameters<typeof imageCompression>[1] = {
    maxSizeMB: 2,
    maxWidthOrHeight: 2048,
    useWebWorker: true,
    fileType: 'image/webp',
    initialQuality: 0.85,
  };

  async compress(file: File): Promise<File> {
    if (!file.type.startsWith('image/')) return file;
    const compressed = await imageCompression(file, this.options);
    const name = file.name.replace(/\.[^.]+$/, '') + '.webp';
    return new File([compressed], name, { type: 'image/webp', lastModified: Date.now() });
  }

  async compressMany(files: File[]): Promise<File[]> {
    return Promise.all(files.map(f => this.compress(f)));
  }
}

export interface Page {
  index: number;
  name: string;
  scan: string;
  scan_width: number;
  scan_height: number;
  thumb: string;
  thumb_width: number;
  thumb_height: number;
  batch: number;
  import_order: number;
}

export interface PageDb {
  pages: Page[];
  next_batch: number;
}

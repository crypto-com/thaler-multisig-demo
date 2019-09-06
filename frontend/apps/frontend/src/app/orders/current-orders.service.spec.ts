import { TestBed } from '@angular/core/testing';

import { CurrentOrdersService } from './current-orders.service';

describe('CurrentOrdersService', () => {
  beforeEach(() => TestBed.configureTestingModule({}));

  it('should be created', () => {
    const service: CurrentOrdersService = TestBed.get(CurrentOrdersService);
    expect(service).toBeTruthy();
  });
});

import { TestBed } from '@angular/core/testing';

import { EscrowMerchantService } from './escrow-merchant.service';

describe('EscrowMerchantService', () => {
  beforeEach(() => TestBed.configureTestingModule({}));

  it('should be created', () => {
    const service: EscrowMerchantService = TestBed.get(EscrowMerchantService);
    expect(service).toBeTruthy();
  });
});

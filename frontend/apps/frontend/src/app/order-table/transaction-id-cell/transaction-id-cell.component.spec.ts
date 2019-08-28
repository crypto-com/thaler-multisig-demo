import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { TransactionIdCellComponent } from './transaction-id-cell.component';

describe('TransactionIdCellComponent', () => {
  let component: TransactionIdCellComponent;
  let fixture: ComponentFixture<TransactionIdCellComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ TransactionIdCellComponent ]
    })
    .compileComponents();
  }));

  beforeEach(() => {
    fixture = TestBed.createComponent(TransactionIdCellComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

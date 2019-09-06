import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { OrderTableactionCellComponent } from './order-tableaction-cell.component';

describe('OrderTableactionCellComponent', () => {
  let component: OrderTableactionCellComponent;
  let fixture: ComponentFixture<OrderTableactionCellComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ OrderTableactionCellComponent ]
    })
    .compileComponents();
  }));

  beforeEach(() => {
    fixture = TestBed.createComponent(OrderTableactionCellComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

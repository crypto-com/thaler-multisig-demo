import { async, ComponentFixture, TestBed } from '@angular/core/testing';

import { MarkOrderFormComponent } from './mark-order-form.component';

describe('MarkOrderFormComponent', () => {
  let component: MarkOrderFormComponent;
  let fixture: ComponentFixture<MarkOrderFormComponent>;

  beforeEach(async(() => {
    TestBed.configureTestingModule({
      declarations: [ MarkOrderFormComponent ]
    })
    .compileComponents();
  }));

  beforeEach(() => {
    fixture = TestBed.createComponent(MarkOrderFormComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
